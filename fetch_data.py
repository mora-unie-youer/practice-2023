#!/usr/bin/env python3
import json
import requests
import time
import threading

from datetime import datetime, timedelta
from pathlib import Path
from typing import Any

#START = datetime(year=2023, month=3, day=8)
START = datetime(year=2023, month=3, day=18)
#N_DAYS = 4 * 7 + 1 - 3
#N_DAYS = 365 # We want to break
#N_DAYS = 2 # Limit
N_DAYS = 1 # Limit
DIR = Path(__file__).resolve().parent / "loaded"
DIR.mkdir(parents=True, exist_ok=True)

BASE_URL = "http://webrobo.mgul.ac.ru:3000/db_api_REST/calibr"
HEADERS = {
    'User-Agent': 'Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/41.0.2228.0 Safari/537.36',
}

class FetchDataThread(threading.Thread):
    def __init__(self, date: datetime):
        threading.Thread.__init__(self)
        self.daemon = True

        self.date = date.strftime("%Y-%m-%d")
        self.filename = date.strftime("%Y-%m-%d.json")
        self.file = DIR / self.filename
        self.url = f"{BASE_URL}/day/{self.date}"

    def run(self):
        while True:
            try:
                print(f"Trying to download {self.filename}...")
                response = requests.get(self.url, headers=HEADERS)
                data = response.json()

                with self.file.open("w") as fp:
                    json.dump(data, fp, indent=2, ensure_ascii=False)
                print(f"File {self.file} saved.")

                return
            except:
                # print(f"Failed to download {self.filename}, retrying in 5 seconds...")
                # time.sleep(5)
                print(f"Failed to download {self.filename}, retrying...")
                # time.sleep(0.5)

threads = [FetchDataThread(START + timedelta(days=i)) for i in range(N_DAYS)]

for thread in threads:
    thread.start()

for thread in threads:
    thread.join()
