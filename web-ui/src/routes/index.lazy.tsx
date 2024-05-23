import { createLazyFileRoute } from "@tanstack/react-router";
import { type Project, useWebsocketContext } from "./../providers/WebsocketProvider";

export const Route = createLazyFileRoute("/")({
    component: Index,
});

function Index() {
    //const { data } = useQuery({ queryKey: queryKey, queryFn: () => {}, staleTime: Infinity });

    const { sendMessage, canSendMessages } = useWebsocketContext() as Project;

    console.log(canSendMessages);
    sendMessage({ username: "test" });

    return (
        <div className="p-2">
            <h3>Welcome Home!</h3>
        </div>
    );
}
