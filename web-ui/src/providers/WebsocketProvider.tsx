import { createContext, useCallback, useContext, useEffect } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { v4 as uuidv4 } from "uuid";
//import { useQueryClient } from "@tanstack/react-query";

export interface WebsocketMessage {
    username: string;
}

// interface WebsocketResponse {
//     username: string;
// }

export type Project = {
    canSendMessages: boolean;
    sendMessage: (message: WebsocketMessage) => void;
};

const WebSocketContext = createContext<Project | undefined>(undefined);

WebSocketContext.displayName = "WebSocketContext";

const SOCKET_URL = "ws://localhost:3000/api/control";
export const queryKey = ["messages"];

export const WebSocketProvider = (props: { children: React.ReactNode }) => {
    const { children } = props;
    //const queryClient = useQueryClient();
    const { sendMessage: sM, lastMessage, readyState } = useWebSocket(SOCKET_URL);

    const canSendMessages = readyState === ReadyState.OPEN;

    useEffect(() => {
        if (lastMessage && lastMessage.data) {
            const { type, payload } = JSON.parse(lastMessage.data);
            console.log(type, payload);

            // queryClient.setQueryData(queryKey, (oldData: WebsocketResponse) => {
            //     return [...oldData, payload];
            // });
        }
    });

    const sendMessage = useCallback(
        (content: WebsocketMessage) => {
            console.log(content);
            if (canSendMessages)
                sM(
                    JSON.stringify({
                        id: uuidv4().toString(),
                        command: { ping: {} },
                    }),
                );
        },
        [canSendMessages, sM],
    );

    return (
        <WebSocketContext.Provider value={{ canSendMessages, sendMessage }}>
            {children}
        </WebSocketContext.Provider>
    );
};

export const useWebsocketContext = () => useContext(WebSocketContext);
