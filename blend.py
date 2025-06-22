
import os.path
import requests

server = "localhost"
url = f"https://{server}:44001/swagger-ui/"

url_up = f"https://{server}:44001/api/v1/todos/upload/"
url_list = f"https://{server}:44001/api/v1/todos/list"
url_blend = f"https://{server}:44001/api/v1/todos/blend"

file_list = [
    "../BlendResult/robot/results/output_a.xml",
    "../BlendResult/robot/results/output_b.xml",
    "../BlendResult/robot/results/output_c.xml",
]

for file in file_list:
    filename = os.path.basename(file)
    files = {'upload_file': open(file, 'rb')}
    r = requests.post(url_up+filename, files=files, verify=False)
    r.raise_for_status()

ret = requests.get(url_list, verify=False)
ret.raise_for_status()
print(ret.text)

ret = requests.get(url_blend, verify=False)
ret.raise_for_status()

# print(ret.text)
with open("out.ods", "w") as f:
    f.write(ret.text)

