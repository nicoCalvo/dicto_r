
import json
import sys
import time


from  dicto_r import dictor as dicto_r
from dictor import dictor

with open('perf.json') as f:
    PERF_JSON_DATA = json.load(f)

def main():
    start_time = time.time()
    lib = sys.argv[1]
    try:
        path = sys.argv[2]
    except:
        path = "email"

    if lib == "py":
        res = dictor(PERF_JSON_DATA, search=path)
    elif lib == "rust":
        res = dicto_r(PERF_JSON_DATA, search=path)
    elif lib == "compare":
        print("compare Ok")
        assert dictor(PERF_JSON_DATA, search=path) == dicto_r(PERF_JSON_DATA, search=path)
    #print(res)
    print("--- %s seconds ---" % (time.time() - start_time))

main()