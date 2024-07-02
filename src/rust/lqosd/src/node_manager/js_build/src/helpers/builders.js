export function heading5Icon(icon, text) {
    let h5 = document.createElement("h5");
    h5.innerHTML = "<i class='fa fa-" + icon + "'></i> " + text;
    return h5;
}

export function theading(text) {
    let th = document.createElement("th");
    th.innerText = text;
    return th;
}

export function simpleRow(text) {
    let td = document.createElement("td");
    td.innerText = text;
    return td;
}

export function clearDashDiv(id, target) {
    let limit = 1;
    if (id.includes("___")) limit = 0;
    while (target.children.length > limit) {
        target.removeChild(target.lastChild);
    }
}