let ws = new WebSocket("ws://localhost:9910");

let interval = null;

ws.onopen = () => {
    interval = setInterval(() => {
        const lights = [];
        for (let i = 0; i < 28; i++) {
            lights[i] = {
                id: i,
                red: Math.floor(Math.random() * 255),
                green: Math.floor(Math.random() * 255),
                blue: Math.floor(Math.random() * 255)
            };
        }

        const msg = createMessage("test", lights);
        ws.send(msg);
    }, 100);
};

ws.onclose = () => {
    if (interval) {
        clearInterval(interval);
        interval = null;
    }
};

/**
 * @typedef {Object} Light
 * @property {number} id Id
 * @property {number} red Red color
 * @property {number} green Green color
 * @property {number} blue Blue color
 */

/**
 * Create a new message buffer.
 * @param {string} nick Nick to use
 * @param {Light[]} lights
 */
function createMessage(nick, lights) {
    const buffer = new Uint8Array(1 + 2 + nick.length + lights.length * 6);

    // protocol version
    buffer[0] = 1;
    // begin nick tag
    buffer[1] = 0;

    let offset = 2;
    for (let i = 0; i < nick.length; i++) {
        buffer[offset++] = nick.codePointAt(i);
    }
    buffer[offset++] = 0;

    for (let i = 0; i < lights.length; i++) {
        const light = lights[i];
        // begin light tag
        buffer[offset] = 1;
        buffer[offset + 1] = light.id;
        // RGB light type
        buffer[offset + 2] = 0;
        buffer[offset + 3] = light.red;
        buffer[offset + 4] = light.green;
        buffer[offset + 5] = light.blue;
        offset += 6;
    }

    return buffer;
}
