# 'fq_codel' or 'cake diffserv4'
# 'cake diffserv4' is recommended
# sqm = 'fq_codel'
sqm = 'cake diffserv4'

# Used to passively monitor the network for before / after comparisons. Leave as False to
# ensure actual shaping. After changing this value, run "sudo systemctl restart LibreQoS.service"
monitorOnlyMode = False

# How many Mbps are available to the edge of this network
upstreamBandwidthCapacityDownloadMbps = 1000
upstreamBandwidthCapacityUploadMbps = 1000

# Devices in ShapedDevices.csv without a defined ParentNode will be placed under a generated
# parent node, evenly spread out across CPU cores. Here, define the bandwidth limit for each
# of those generated parent nodes.
generatedPNDownloadMbps = 1000
generatedPNUploadMbps = 1000

# Interface connected to core router
interfaceA = 'eth1'

# Interface connected to edge router
interfaceB = 'eth2'

# Allow shell commands. False causes commands print to console only without being executed.
# MUST BE ENABLED FOR PROGRAM TO FUNCTION
enableActualShellCommands = True

# Add 'sudo' before execution of any shell commands. May be required depending on distribution and environment.
runShellCommandsAsSudo = False

# Allows overriding queues / CPU cores used. When set to 0, the max possible queues / CPU cores are utilized. Please leave as 0.
queuesAvailableOverride = 0

# Some networks are flat - where there are no Parent Nodes defined in ShapedDevices.csv
# For such flat networks, just define network.json as {} and enable this setting
# By default, it balances the subscribers across CPU cores, factoring in their max bandwidth rates
# Past 25,000 subsribers this algorithm becomes inefficient and is not advised
useBinPackingToBalanceCPU = True

# Bandwidth & Latency Graphing
influxDBEnabled = True
influxDBurl = "http://localhost:8086"
influxDBBucket = "libreqos"
influxDBOrg = "Your ISP Name Here"
influxDBtoken = ""

# NMS/CRM Integration

# If a device shows a WAN IP within these subnets, assume they are behind NAT / un-shapable, and ignore them
ignoreSubnets = ['192.168.0.0/16']
allowedSubnets = ['100.64.0.0/10']

# Splynx Integration
automaticImportSplynx = False
splynx_api_key = ''
splynx_api_secret = ''
# Everything before /api/2.0/ on your Splynx instance
splynx_api_url = 'https://YOUR_URL.splynx.app'

# UISP integration
automaticImportUISP = False
uispAuthToken = ''
# Everything before /nms/ on your UISP instance
UISPbaseURL = 'https://examplesite.com'
# UISP Site - enter the name of the root site in your network tree
# to act as the starting point for the tree mapping
uispSite = ''
# Strategy:
# * "flat" - create all client sites directly off the top of the tree,
#   provides maximum performance - at the expense of not offering AP,
#   or site options.
# * "full" - build a complete network map
uispStrategy = "full"
# List any sites that should not be included, with each site name surrounded by '' and seperated by commas
excludeSites = []
# If you use IPv6, this can be used to find associated IPv6 prefixes for your clients' IPv4 addresses, and match them to those devices
findIPv6usingMikrotik = False
# If you want to provide a safe cushion for speed test results to prevent customer complains, you can set this to 1.15 (15% above plan rate).
# If not, you can leave as 1.0
bandwidthOverheadFactor = 1.0
# For edge cases, set the respective ParentNode for these CPEs
exceptionCPEs = {}
                #  'CPE-SomeLocation1': 'AP-SomeLocation1',
                #  'CPE-SomeLocation2': 'AP-SomeLocation2',
                #}

# API Auth
apiUsername = "testUser"
apiPassword = "changeme8343486806"
apiHostIP = "127.0.0.1"
apiHostPost = 5000
