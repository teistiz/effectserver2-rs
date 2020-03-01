"use strict";

(function() {
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
        setStatus("connecting");
        const url = new URL(document.URL);
        socket = new WebSocket(`ws://${url.host}/`);
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
        block.id = `light-${id}`;
        block.className="light";
        return block;
    }

    function updateBlock(element, lightStatus) {
        const { id, r, g, b } = lightStatus;
        element.innerText = id;
        element.style = `background: rgb(${r}, ${g}, ${b})`;
    }

    openSocket();
})();
