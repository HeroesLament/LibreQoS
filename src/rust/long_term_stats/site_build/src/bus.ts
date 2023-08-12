import { is_wasm_connected, send_wss_queue, initialize_wss } from "../wasm/wasm_pipe";
import { Auth } from "./auth";
import { SiteRouter } from "./router";

export class Bus {
    ws: WebSocket;

    constructor() {
        const currentUrlWithoutAnchors = window.location.href.split('#')[0].replace("https://", "").replace("http://", "");
        if (window.location.href.startsWith("https://")) {
            const url = "wss://" + currentUrlWithoutAnchors + "ws";
            initialize_wss(url);
        } else {
            const url = "ws://" + currentUrlWithoutAnchors + "ws";
            initialize_wss(url);
        }   
    }

    updateConnected() {
        //console.log("Connection via WASM: " + is_wasm_connected());
        let indicator = document.getElementById("connStatus");
        if (indicator && is_wasm_connected()) {
            indicator.style.color = "green";

            // Clear the loader
            let loader = document.getElementById('SpinLoad');
            if (loader) {
                loader.style.display = "none";
            }
        } else if (indicator) {
            indicator.style.color = "red";
        }
    }

    sendQueue() {
        send_wss_queue();
    }

    getToken(): string {
        if (window.auth.hasCredentials && window.auth.token) {
            return window.auth.token;
        } else {
            return "";
        }
    }

    requestThroughputChartCircuit(circuit_id: string) {
        let request = {
            msg: "throughputChartCircuit",
            period: window.graphPeriod,
            circuit_id: decodeURI(circuit_id),
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }

    requestThroughputChartSite(site_id: string) {
        let request = {
            msg: "throughputChartSite",
            period: window.graphPeriod,
            site_id: decodeURI(site_id),
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }

    requestRttChartSite(site_id: string) {
        let request = {
            msg: "rttChartSite",
            period: window.graphPeriod,
            site_id: decodeURI(site_id),
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }

    requestRttChartCircuit(circuit_id: string) {
        let request = {
            msg: "rttChartCircuit",
            period: window.graphPeriod,
            circuit_id: decodeURI(circuit_id),
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }

    requestSiteHeat(site_id: string) {
        let request = {
            msg: "siteHeat",
            period: window.graphPeriod,
            site_id: decodeURI(site_id),
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }

    sendSearch(term: string) {
        let request = {
            msg: "search",
            term: term,
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }

    requestSiteInfo(site_id: string) {
        let request = {
            msg: "siteInfo",
            site_id: decodeURI(site_id),
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }

    requestCircuitInfo(circuit_id: string) {
        let request = {
            msg: "circuitInfo",
            circuit_id: decodeURI(circuit_id),
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }

    requestSiteParents(site_id: string) {
        let request = {
            msg: "siteParents",
            site_id: decodeURI(site_id),
        };
        let json = JSON.stringify(request);
        this.ws.send(json);
    }
}

// WASM callback
export function onAuthFail() {
    window.auth.hasCredentials = false;
    window.login = null;
    window.auth.token = null;
    localStorage.removeItem("token");
    window.router.goto("login");
}

// WASM callback
export function onAuthOk(token: string, name: string, license_key: string) {
    window.auth.hasCredentials = true;
    window.login = { msg: "authOk", token: token, name: name, license_key: license_key };
    window.auth.token = token;
}

// WASM Callback
export function onMessage(rawJson: string) {
    let json = JSON.parse(rawJson);
    //console.log(json);
    //console.log(Object.keys(json));
    json.msg = Object.keys(json)[0];
    window.router.onMessage(json);   
}