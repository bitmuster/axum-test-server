
import os.path
import requests

server = "localhost"
url = f"https://{server}:44001/swagger-ui/"

url_up = f"https://{server}:44001/api/v1/blend/upload/"
url_list = f"https://{server}:44001/api/v1/blend/list"
url_blend = f"https://{server}:44001/api/v1/blend/blend"

file_list = [
    "../BlendResult/robot/results/output_a.xml",
    "../BlendResult/robot/results/output_b.xml",
    "../BlendResult/robot/results/output_c_fail.xml",
]

headers = {'theapikey': 'rocks'}

verify=False

for file in file_list:
    filename = os.path.basename(file)
    files = {'upload_file': open(file, 'rb')}
    r = requests.post(url_up+filename, files=files, verify=verify, headers=headers)
    r.raise_for_status()

ret = requests.get(url_list, verify=verify, headers=headers)
ret.raise_for_status()
print(ret.text)

ret = requests.get(url_blend, verify=verify, headers=headers)
ret.raise_for_status()

# print(ret.text)
with open("out.ods", "wb") as f:
    f.write(ret.content)

