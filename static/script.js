"use strict";

(function () {
    const eRoot = document.getElementById("app");

    const eConnStatus = document.createElement("div");
    eConnStatus.className = "status";
    eRoot.append(eConnStatus);

    const eLights = document.createElement("div");
    eLights.className = "lights";
    eRoot.append(eLights);

    /** @type {WebSocket} */
    let socket;

    /**
     * Set the current status message.
     * @param {string} statusMessage
     */
    function setStatus(statusMessage) {
        eConnStatus.innerHTML = statusMessage;
    }

    /**
     * Open a new WebSocket.
     */
    function openSocket() {
        if (socket) {
            socket.close();
            socket = null;
        }
        const documentUrl = new URL(document.URL);
        const url = `ws://${documentUrl.host}/ws`
        setStatus(`connecting to ${url}`);
        socket = new WebSocket(url);
        socket.onopen = (event) => {
            console.info("[ws] Open:", event);
            setStatus("connected");
        };
        socket.onclose = () => {
            setStatus("closed");
            setTimeout(openSocket, 1000);
        };
        socket.onerror = (event) => {
            console.error("[ws] Error:", event);
        };
        socket.onmessage = event => {
            handleMessage(JSON.parse(event.data));
        };
    }

    function handleMessage(msg) {
        eLights.innerText = JSON.stringify(msg, null, 4);
        updateLights(msg.lights);
    }

    let lastCount = 0;

    /**
     * Update the lights from a WS message.
     * @param {any[]} lights
     */
    function updateLights(lights) {
        const rebuild = lastCount !== lights.length;

        if (rebuild) {
            eLights.innerHTML = "";
        }

        for (const light of lights) {
            const elementId = `light-${light.id}`;
            const element = rebuild ? createBlock(elementId) : document.getElementById(elementId);
            updateBlock(element, light);
            if (rebuild) {
                eLights.append(element);
            }
        }
    }

    function createBlock(id) {
        const block = document.createElement("div");
        block.id = id;
        block.className = "light";

        const makeChild = (tag, className) => {
            const element = document.createElement(tag);
            element.className = className;
            block.appendChild(element);
            return element;
        };
        block.__id = makeChild("div", "light__id");
        block.__tag = makeChild("div", "light__tag");
        block.__ip = makeChild("div", "light__ip");

        return block;
    }

    function updateBlock(element, lightStatus) {
        const { id, r, g, b } = lightStatus;
        element.style = `background: rgb(${r}, ${g}, ${b})`;
        element.__id.innerText = id;
        element.__ip.innerText = lightStatus.ip;
        element.__tag.innerText = lightStatus.tag;
    }

    openSocket();
})();
