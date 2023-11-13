#!/usr/bin/python3
import json
import os
from websocket import create_connection

CONF = {"port": 1234, "vol_file": "hello"}
CAMILLA_PORT_PREFIX = "-p"
CAMILLA_VOL_FILE_PREFIX = "vol_file"


def _get_line(prefix: str, line: str) -> str:
    return line.replace(prefix, "").replace(" ", "").replace('"', "")


def main(location_of_asoundrc: str):
    with open(location_of_asoundrc) as f:
        for line in f.readlines():
            trimmed_line = line.strip()
            if trimmed_line.startswith(CAMILLA_PORT_PREFIX):
                CONF["port"] = int(_get_line(CAMILLA_PORT_PREFIX, trimmed_line))
            if trimmed_line.startswith(CAMILLA_VOL_FILE_PREFIX):
                CONF["vol_file"] = _get_line(CAMILLA_VOL_FILE_PREFIX, trimmed_line)

    ws = create_connection(f"ws://127.0.0.1:{CONF['port']}")
    ws.send(json.dumps("GetVolume"))
    message = ws.recv()

    with open(CONF["vol_file"], "x") as f:
        f.write(f"{message['GetVolume']['value']} 0")


if __name__ == "__main__":
    location = os.path.expanduser("~/.asoundrc")
    main(location)
