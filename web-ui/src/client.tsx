import { create } from "zustand";
import { v4 as uuidv4 } from "uuid";

interface WebSocket {
    onclose: ((event: CloseEvent) => void) | null;
    onerror: ((event: Event) => void) | null;
    onmessage: ((event: MessageEvent) => void) | null;
    onopen: ((event: Event) => void) | null;
    close(code?: number, reason?: string): void;
    send(data: string | ArrayBuffer | Blob | ArrayBufferView): void;
    readonly readyState: WebSocket | number;
}

enum State {
    Null,
    Playing,
    Stopped,
    Ready,
}

interface Device {
    id: string;
    device_num: number;
    state: State;
}

interface ClientState {
    connected: boolean;
    devices: Device[];
    setConnnected: (by: boolean) => void;
    setDevices: (devices: Device[]) => void;
}

export const useClientState = create<ClientState>((set) => ({
    connected: false,
    devices: [],
    setConnnected: (connected: boolean) => set({ connected }),
    setDevices: (devices: Device[]) => set({ devices }),
}));

export class Client {
    static shared: Client = new Client();

    url = "ws://10.0.10.41:3000/api/control";
    ws: WebSocket | undefined;
    reconnectTimer: number | undefined;

    connect() {
        if (this.ws) {
            this.ws.close();
        }

        this.ws = new WebSocket(this.url);
        this.ws.onopen = this.onOpen.bind(this);
        this.ws.onmessage = this.onMessage.bind(this);
        this.ws.onclose = this.onClose.bind(this);
    }

    onOpen() {
        console.log("Connected to server");
        useClientState.getState().setConnnected(true);
        clearTimeout(this.reconnectTimer);
    }

    onClose() {
        console.log("Disconnected from server");
        useClientState.getState().setConnnected(false);
        this.reconnectTimer = setTimeout(() => this.connect(), 1000);
    }

    onMessage(event: MessageEvent) {
        let message;
        try {
            message = JSON.parse(event.data);
        } catch (e) {
            console.log("Error parsing message", event.data);
            return;
        }

        if (Object.prototype.hasOwnProperty.call(message.result, "sync")) {
            return useClientState.getState().setDevices(message.result.sync as [Device]);
        }
    }

    send(message: string) {
        if (this.ws?.readyState === WebSocket.OPEN && useClientState.getState().connected) {
            return this.ws?.send(message);
        }
        console.log("Websocket not open, not sending message", message);
        setTimeout(() => this.send(message), 1000);
    }

    start(device_id: string) {
        this.send(
            JSON.stringify({
                id: uuidv4().toString(),
                command: { start: { device_id: device_id } },
            }),
        );
    }

    stop(device_id: string) {
        this.send(
            JSON.stringify({
                id: uuidv4().toString(),
                command: { stop: { device_id: device_id } },
            }),
        );
    }

    sync() {
        this.send(
            JSON.stringify({
                id: uuidv4().toString(),
                command: { sync: {} },
            }),
        );
    }

    ping() {
        this.send(
            JSON.stringify({
                id: uuidv4().toString(),
                command: { ping: {} },
            }),
        );
    }
}
