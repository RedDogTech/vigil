import { createLazyFileRoute } from "@tanstack/react-router";
import Button from "react-bootstrap/Button";
import Table from "react-bootstrap/Table";
import Badge from "react-bootstrap/Badge";
import ButtonGroup from "react-bootstrap/ButtonGroup";
import { useEffect } from "react";
import { Client, useClientState } from "../client";

export const Route = createLazyFileRoute("/")({
    component: Index,
});

function Index() {
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
                                <td>
                                    <Badge bg="secondary">{device.state}</Badge>
                                </td>
                                <td>
                                    <ButtonGroup>
                                        <Button variant="primary" onClick={() => start(device.id)}>
                                            Start
                                        </Button>
                                        <Button variant="danger" onClick={() => stop(device.id)}>
                                            Stop
                                        </Button>
                                    </ButtonGroup>
                                </td>
                            </tr>
                        );
                    })}
                </tbody>
            </Table>
        </div>
    );
}
