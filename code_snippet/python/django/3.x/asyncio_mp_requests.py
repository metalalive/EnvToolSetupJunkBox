import asyncio
import requests
from multiprocessing.dummy import Pool
from datetime import datetime


pool = Pool(2)
loop = asyncio.get_event_loop()

async def testcall_async(url):
    print('start with the url: %s' % url)
    response = requests.get(url)
    print('end with the url: %s , status: %s, content-length: %s' % \
            (url, response.status_code, len(response.content)))

def testcall(url):
    print('start with the url: %s' % url)
    response = requests.get(url)
    print('end with the url: %s , status: %s, content-length: %s' % \
            (url, response.status_code, len(response.content)))


def main_v2(*urls):
    print('start of main_v2, %s' % datetime.now())
    futures = []
    for u in urls:
        f = testcall_async(u)
        futures.append(f)
    ga = asyncio.gather( *futures )
    loop.run_until_complete(ga)
    print('end of main_v2, %s\n' % datetime.now())


def main_v3(*urls):
    futures = []
    for u in urls:
        future = pool.apply_async(testcall, [u])
        futures.append(future)
    print('start of main_v3, %s' % datetime.now())
    for future in futures:
        future.get()
    print('end of main_v3, %s\n' % datetime.now())


u1 = 'http://localhost:8008/quota_material'
u2 = 'http://www.pochang.com/blog/2006/04/18/echobandcom-podcast/'
main_v2(u2, u1)
main_v3(u2, u1)


