import { createLazyFileRoute } from "@tanstack/react-router";
import Button from "react-bootstrap/Button";
import Table from "react-bootstrap/Table";
import { useEffect } from "react";
import { Client, useClientState } from "../client";

export const Route = createLazyFileRoute("/")({
    component: Index,
});

function Index() {
    useEffect(() => {
        Client.shared.connect();
        //generate();
    }, []);

    const start = (device_id: string) => {
        Client.shared.start(device_id);
    };

    const stop = (device_id: string) => {
        Client.shared.stop(device_id);
    };

    const { devices } = useClientState();

    return (
        <div className="p-2">
            <Table bordered hover>
                <thead>
                    <tr>
                        <th>#</th>
                        <th>Device id</th>
                        <th>State</th>
                        <th>Control</th>
                    </tr>
                </thead>
                <tbody>
                    {devices.map((device) => {
                        return (
                            <tr>
                                <td>{device.device_num}</td>
                                <td>{device.id}</td>
                                <td>{device.state}</td>
                                <td>
                                    {" "}
                                    <Button variant="primary" onClick={() => start(device.id)}>
                                        Start
                                    </Button>{" "}
                                    <Button variant="danger" onClick={() => stop(device.id)}>
                                        Stop
                                    </Button>{" "}
                                </td>
                            </tr>
                        );
                    })}
                </tbody>
            </Table>
        </div>
    );
}
