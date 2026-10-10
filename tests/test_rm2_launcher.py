"""Exercise actual launcher control flow against fake hardware/services."""
import hashlib
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]

class LauncherTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.d = Path(self.tmp.name)
        self.log = self.d / 'log'
        (self.d / 'machine').write_text('reMarkable 2.0\n')
        (self.d / 'update.conf').write_text('REMARKABLE_RELEASE_VERSION=3.0.4.1305\n')
        self.exe('systemctl', '#!/bin/sh\necho "systemctl $*" >> "$TEST_LOG"\nif [ "$1" = is-active ]; then exit "${TEST_XOCHITL_STATUS:-0}"; fi\nexit 0\n')
        self.exe('pgrep', '#!/bin/sh\nexit "${TEST_OTHER_SERVER:-1}"\n')
        self.exe('id', '#!/bin/sh\necho 0\n')
        self.exe('server', '''#!/bin/sh
trap 'exit 0' TERM INT
echo server >> "$TEST_LOG"
case "${TEST_SERVER_MODE:-alive}" in
  dead) exit 9;;
  crash) sleep 1; exit 9;;
esac
while :; do sleep 0.1; done
''')
        self.exe('riddle', '''#!/bin/sh
if [ "${1:-}" = --rm2-check ]; then exit "${TEST_CHECK_STATUS:-0}"; fi
echo app >> "$TEST_LOG"
if [ "${TEST_APP_MODE:-exit}" = wait ]; then
    trap 'exit 143' TERM INT
    while :; do sleep 0.1; done
fi
exit "${TEST_APP_STATUS:-0}"
''')
        (self.d / 'librm2fb_server.so.1.0.1').write_bytes(b'fake server')
        (self.d / 'oracle.env').write_text("RIDDLE_OPENAI_KEY='fake-test-key'\n")
        script = (ROOT / 'scripts/rm2/run.sh').read_text()
        # Substitute filesystem fixtures and fake server; test never touches host services.
        for old,new in [('/sys/devices/soc0/machine',str(self.d/'machine')),('/usr/share/remarkable/update.conf',str(self.d/'update.conf')),('/run/riddle-rm2.lock',str(self.d/'lock')),('/run/riddle-rm2.restore',str(self.d/'restore')),('/usr/bin/xochitl',str(self.d/'server'))]:
            script=script.replace(old,new)
        # The stub is a script, not an ELF; don't ask the host loader to preload it.
        script=script.replace('LD_PRELOAD="$HERE/librm2fb_server.so.1.0.1" ', '')
        self.exe('run.sh',script)
        self.checksums()
        self.env=dict(os.environ,PATH=f'{self.d}:{os.environ["PATH"]}',TEST_LOG=str(self.log),INVOCATION_ID='fixture')
    def exe(self,name,body):
        p=self.d/name;p.write_text(body);p.chmod(0o755)
    def checksums(self):
        (self.d/'SHA256SUMS').write_text(''.join(f'{hashlib.sha256((self.d/n).read_bytes()).hexdigest()}  {n}\n' for n in ['riddle','run.sh','librm2fb_server.so.1.0.1']))
    def run_script(self,*args,**env):
        return subprocess.run([str(self.d/'run.sh'),*args],env=self.env|env,capture_output=True,text=True,timeout=12)
    def lines(self):
        return self.log.read_text().splitlines() if self.log.exists() else []
    def test_stop_hook_only_restores_its_own_session(self):
        script=(ROOT/'scripts/rm2/recover.sh').read_text().replace('/run/riddle-rm2.restore',str(self.d/'restore'))
        self.exe('recover.sh',script)
        for owner in [None, 'someone-else']:
            if owner: (self.d/'restore').write_text(owner)
            r=subprocess.run([str(self.d/'recover.sh')],env=self.env,capture_output=True)
            self.assertEqual(r.returncode,0)
            self.assertEqual(self.lines(),[])
        (self.d/'restore').write_text('fixture')
        r=subprocess.run([str(self.d/'recover.sh')],env=self.env,capture_output=True)
        self.assertEqual(r.returncode,0)
        self.assertEqual(self.lines(),['systemctl --no-block start xochitl'])
        self.assertFalse((self.d/'restore').exists())

    def test_preflight_is_read_only_for_services(self):
        r=self.run_script('--check');self.assertEqual(r.returncode,0,r.stderr)
        self.assertEqual(self.lines(),[])
    def test_wrong_firmware_never_stops_interface(self):
        (self.d/'update.conf').write_text('REMARKABLE_RELEASE_VERSION=3.1.0.1346\n')
        self.assertNotEqual(self.run_script().returncode,0)
        self.assertEqual(self.lines(),[])
    def test_wrong_model_never_stops_interface(self):
        (self.d/'machine').write_text('reMarkable 1.0\n')
        self.assertNotEqual(self.run_script().returncode,0)
        self.assertEqual(self.lines(),[])
    def test_missing_or_placeholder_key_never_stops_interface(self):
        (self.d/'oracle.env').write_text("RIDDLE_OPENAI_KEY='REPLACE_WITH_YOUR_API_KEY'\n")
        self.assertNotEqual(self.run_script().returncode,0)
        self.assertEqual(self.lines(),[])
    def test_bad_checksum_never_stops_interface(self):
        (self.d/'librm2fb_server.so.1.0.1').write_bytes(b'changed')
        self.assertNotEqual(self.run_script().returncode,0)
        self.assertEqual(self.lines(),[])
    def test_competing_server_never_stops_interface(self):
        self.assertNotEqual(self.run_script(TEST_OTHER_SERVER='0').returncode,0)
        self.assertEqual(self.lines(),[])
    def test_inactive_interface_is_not_taken_over_or_restarted(self):
        r=self.run_script(TEST_XOCHITL_STATUS='3')
        self.assertNotEqual(r.returncode,0)
        self.assertNotIn('systemctl stop xochitl',self.lines())
        self.assertNotIn('systemctl start xochitl',self.lines())
        self.assertFalse((self.d/'restore').exists())
    def test_normal_exit_restores_interface(self):
        r=self.run_script();self.assertEqual(r.returncode,0,r.stderr)
        self.assertIn('app',self.lines())
        self.assertIn('systemctl stop xochitl',self.lines())
        self.assertEqual(self.lines()[-1],'systemctl start xochitl')
    def test_app_crash_restores_and_preserves_exit_status(self):
        r=self.run_script(TEST_APP_STATUS='42');self.assertEqual(r.returncode,42,r.stderr)
        self.assertEqual(self.lines()[-1],'systemctl start xochitl')
    def test_server_crash_during_idle_restores_interface(self):
        r=self.run_script(TEST_SERVER_MODE='crash',TEST_APP_MODE='wait')
        self.assertNotEqual(r.returncode,0,r.stderr)
        self.assertEqual(self.lines()[-1],'systemctl start xochitl')
    def test_termination_restores_interface(self):
        p=subprocess.Popen([str(self.d/'run.sh')],env=self.env|{'TEST_APP_MODE':'wait'},stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        try:
            deadline=time.monotonic()+4
            while 'app' not in self.lines() and time.monotonic()<deadline: time.sleep(.02)
            self.assertIn('app',self.lines())
            p.send_signal(signal.SIGTERM)
            self.assertEqual(p.wait(timeout=5),143)
            self.assertEqual(self.lines()[-1],'systemctl start xochitl')
        finally:
            if p.poll() is None: p.kill();p.wait()

if __name__ == '__main__': unittest.main()
