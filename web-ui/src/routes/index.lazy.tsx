import { createLazyFileRoute } from "@tanstack/react-router";
import Button from "react-bootstrap/Button";
import Table from "react-bootstrap/Table";
import { useEffect } from "react";
import { Client } from "../client";

export const Route = createLazyFileRoute("/")({
    component: Index,
});

function Index() {
    const generate = () => {
        Client.shared.sync();
    };

    useEffect(() => {
        Client.shared.connect();
        //generate();
    }, []);

    const start = (num: number) => {
        Client.shared.start(num);
    };

    const stop = (num: number) => {
        Client.shared.stop(num);
    };

    return (
        <div className="p-2">
            <Button variant="primary" onClick={generate}>
                Sync
            </Button>{" "}
            <Table bordered hover>
                <thead>
                    <tr>
                        <th>#</th>
                        <th>First Name</th>
                        <th>Last Name</th>
                        <th>Username</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>1</td>
                        <td>Mark</td>
                        <td>Otto</td>
                        <td>
                            {" "}
                            <Button variant="primary" onClick={() => start(1)}>
                                Start
                            </Button>{" "}
                            <Button variant="danger" onClick={() => stop(1)}>
                                Stop
                            </Button>{" "}
                        </td>
                    </tr>
                </tbody>
            </Table>
        </div>
    );
}
