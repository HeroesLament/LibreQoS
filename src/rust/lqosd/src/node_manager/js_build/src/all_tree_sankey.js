import { DashboardGraph } from "./graphs/dashboard_graph";
import {lerpGreenToRedViaOrange} from "./helpers/scaling";
import {isRedacted} from "./helpers/redact";

var allNodes = [];
let rootId = 0;

class AllTreeSankey extends DashboardGraph {
    constructor(id) {
        super(id);
        this.option = {
            series: [
                {
                    type: 'sankey',
                    data: [],
                    links: []
                }
            ]
        };
        this.option && this.chart.setOption(this.option);
        this.chart.hideLoading();
        this.chart.on('click', (params) => {
            //console.log(params.name);
            let name = params.name;
            // If it contains a >, it's a link
            if (name.indexOf(" > ") === -1) {
                for (let i=0; i<allNodes.length; i++) {
                    if (allNodes[i][1].name === name) {
                        rootId = i;
                        break;
                    }
                }
            }
        });
        $("#btnRoot").click(() => { rootId = 0; });
    }

    update(data, links) {
        this.option.series[0].data = data;
        this.option.series[0].links = links;
        this.chart.hideLoading();
        this.chart.setOption(this.option);
    }
}

let lastRtt = {};

function start() {
    $.get("/local-api/networkTree", (data) => {
        allNodes = data;
        //console.log(data);
        let redact = isRedacted();
        $("#rootNode").text(data[rootId][1].name);

        let nodes = [];
        let links = [];

        for (let i=0; i<data.length; i++) {
            let depth = data[i][1].parents.length;
            if (depth > maxDepth) {
                continue;
            }
            // If data[i][1].parents does not contain rootId, skip
            if (!data[i][1].parents.includes(rootId)) {
                continue;
            }
            let name = data[i][1].name;
            let bytes = data[i][1].current_throughput[0];
            let bytesAsMegabits = bytes / 1000000;
            let maxBytes = data[i][1].max_throughput[0] / 8;
            let percent = Math.min(100, (bytesAsMegabits / maxBytes) * 100);
            let capacityColor = lerpGreenToRedViaOrange(100 - percent, 100);


            if (data[i][1].rtts.length > 0) {
                lastRtt[name] = data[i][1].rtts[0];
            } else {
                lastRtt[name] = 0;
            }
            let color = lerpGreenToRedViaOrange(200 - lastRtt[name], 200);

            let label = {
                fontSize: 6,
                color: "#999"
            };
            if (redact) label.fontSize = 0;

            nodes.push({
                name: data[i][1].name,
                label: label,
                itemStyle: {
                    color: color
                }
            });

            if (i > 0) {
                let immediateParent = data[i][1].immediate_parent;
                links.push({
                    source: data[immediateParent][1].name,
                    target: data[i][1].name,
                    value: data[i][1].current_throughput[0] + data[i][1].current_throughput[1],
                    lineStyle: {
                        color: capacityColor,
                    }
                });
            }
        }

        graph.update(nodes, links);
        setTimeout(start, 1000);
    });
}

function getMaxDepth() {
    let maxDepth = 10;
    let storedDepth = localStorage.getItem("atsDepth");
    if (storedDepth !== null) {
        maxDepth = parseInt(storedDepth);
    } else {
        localStorage.setItem("atsDepth", maxDepth.toString());
    }
    return maxDepth;
}

function bindMaxDepth() {
    let d = document.getElementById("maxDepth");
    d.value = maxDepth;
    d.onclick = () => {
        maxDepth = parseInt(d.value);
        localStorage.setItem("atsDepth", maxDepth.toString());
    };
}

let maxDepth = getMaxDepth();
bindMaxDepth();
let graph = new AllTreeSankey("sankey");

start();