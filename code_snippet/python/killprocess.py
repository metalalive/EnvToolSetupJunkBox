import signal
import os
import json

class KillProcessSample:
    def start(self, argv):
        assert len(argv) == 1, "arguments must include config file"
        setting_path  = argv[0]
        pid = -1
        with open(setting_path, 'r') as f:
            cfg_root = json.load(f)
            pid_filep = cfg_root['pid_file']
            with open(pid_filep, 'r') as f2:
                pid = int(f2.readline())
        if pid > 2:
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError  as e:
                print('failed to kill process, PID {_pid} not found'.format(_pid=pid))
