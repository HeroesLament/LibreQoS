from pythonCheck import checkPythonVersion
checkPythonVersion()
import requests
import warnings
from liblqos_python import exclude_sites, find_ipv6_using_mikrotik, bandwidth_overhead_factor, splynx_api_key, \
	splynx_api_secret, splynx_api_url
from integrationCommon import isIpv4Permitted
import base64
from requests.auth import HTTPBasicAuth
if find_ipv6_using_mikrotik() == True:
	from mikrotikFindIPv6 import pullMikrotikIPv6  
from integrationCommon import NetworkGraph, NetworkNode, NodeType

def buildHeaders():
	credentials = splynx_api_key() + ':' + splynx_api_secret()
	credentials = base64.b64encode(credentials.encode()).decode()
	return {'Authorization' : "Basic %s" % credentials}

def spylnxRequest(target, headers):
	# Sends a REST GET request to Spylnx and returns the
	# result in JSON
	url = splynx_api_url() + "/api/2.0/" + target
	r = requests.get(url, headers=headers, timeout=10)
	return r.json()

def getTariffs(headers):
	data = spylnxRequest("admin/tariffs/internet", headers)
	tariff = []
	downloadForTariffID = {}
	uploadForTariffID = {}
	for tariff in data:
		tariffID = tariff['id']
		speed_download = round((int(tariff['speed_download']) / 1000))
		speed_upload = round((int(tariff['speed_upload']) / 1000))
		downloadForTariffID[tariffID] = speed_download
		uploadForTariffID[tariffID] = speed_upload
	return (tariff, downloadForTariffID, uploadForTariffID)

def getCustomers(headers):
	data = spylnxRequest("admin/customers/customer", headers)
	#addressForCustomerID = {}
	#customerIDs = []
	#for customer in data:
	#	customerIDs.append(customer['id'])
	#	addressForCustomerID[customer['id']] = customer['street_1']
	return data

def getRouters(headers):
	data = spylnxRequest("admin/networking/routers", headers)
	ipForRouter = {}
	for router in data:
		routerID = router['id']
		ipForRouter[routerID] = router['ip']
	
	print("Router IPs found: " + str(len(ipForRouter)))
	return ipForRouter

def combineAddress(json):
	# Combines address fields into a single string
	# The API docs seem to indicate that there isn't a "state" field?
	if json["street_1"]=="" and json["city"]=="" and json["zip_code"]=="":
		return json["id"] + "/" + json["name"]
	else:
		return json["street_1"] + " " + json["city"] + " " + json["zip_code"]

def createShaper():
	net = NetworkGraph()

	print("Fetching data from Spylnx")
	headers = buildHeaders()
	tariff, downloadForTariffID, uploadForTariffID = getTariffs(headers)
	customers = getCustomers(headers)
	ipForRouter = getRouters(headers)

	# It's not very clear how a service is meant to handle multiple
	# devices on a shared tariff. Creating each service as a combined
	# entity including the customer, to be on the safe side.
	for customerJson in customers:
		if customerJson['status'] == 'active':
			services = spylnxRequest("admin/customers/customer/" + customerJson["id"] + "/internet-services", headers)
			for serviceJson in services:
				if (serviceJson['status'] == 'active'):
					combinedId = "c_" + str(customerJson["id"]) + "_s_" + str(serviceJson["id"])
					tariff_id = serviceJson['tariff_id']
					customer = NetworkNode(
						type=NodeType.client,
						id=combinedId,
						displayName=customerJson["name"],
						address=combineAddress(customerJson),
						customerName=customerJson["name"],
						download=downloadForTariffID[tariff_id],
						upload=uploadForTariffID[tariff_id],
					)
					net.addRawNode(customer)
					
					ipv4 = ''
					ipv6 = ''
					routerID = serviceJson['router_id']
					# If not "Taking IPv4" (Router will assign IP), then use router's set IP
					# Debug
					taking_ipv4 = int(serviceJson['taking_ipv4'])
					if taking_ipv4 == 0:
						try:
							ipv4 = ipForRouter[routerID]
						except:
							warnings.warn("taking_ipv4 was 0 for client " + combinedId + " but router ID was not found in ipForRouter", stacklevel=2)
							ipv4 = ''
					elif taking_ipv4 == 1:
						ipv4 = serviceJson['ipv4']
						
					# If not "Taking IPv6" (Router will assign IP), then use router's set IP
					if isinstance(serviceJson['taking_ipv6'], str):
						taking_ipv6 = int(serviceJson['taking_ipv6'])
					else:
						taking_ipv6 = serviceJson['taking_ipv6']
					if taking_ipv6 == 0:
						ipv6 = ''
					elif taking_ipv6 == 1:
						ipv6 = serviceJson['ipv6']
					
					device = NetworkNode(
						id=combinedId+"_d" + str(serviceJson["id"]),
						displayName=serviceJson["id"],
						type=NodeType.device,
						parentId=combinedId,
						mac=serviceJson["mac"],
						ipv4=[ipv4],
						ipv6=[ipv6]
					)
					net.addRawNode(device)

	net.prepareTree()
	net.plotNetworkGraph(False)
	if net.doesNetworkJsonExist():
		print("network.json already exists. Leaving in-place.")
	else:
		net.createNetworkJson()
	net.createShapedDevices()

def importFromSplynx():
	#createNetworkJSON()
	createShaper()

if __name__ == '__main__':
	importFromSplynx()
