import { Button, Space, Table } from "antd";
import { ColumnsType } from "antd/es/table";
import React from "react";
import { useQuery } from "./useQuery";

// rome-ignore lint/suspicious/noExplicitAny: <explanation>
const columns: ColumnsType<any> = [
	{
		title: "Name",
		dataIndex: ["args", "Left", "name"],
		key: "name",
	},
	{
		title: "Branch",
		dataIndex: ["args", "Left", "repo_args", "branch"],
		key: "branch",
	},
	{
		title: "Schedule",
		dataIndex: "schedule",
		key: "schedule",
	},
	{
		title: "Whitelist",
		dataIndex: ["args", "Left", "repo_args", "whitelist"],
		key: "schedule",
	},
	{
		title: "Action",
		key: "action",
		render: (_, record) => (
			<Space size="middle">
				<a>Invite {record.name}</a>
				<a>Delete</a>
			</Space>
		),
	},
];

const RepoManage: React.FC = () => {
	const { data, loading } = useQuery("/api/repo/list", { method: "POST" });
	return (
		<div className="flex">
			<div className="flex" style={{ flexDirection: "row" }}>
				<Button>Add</Button>
			</div>
			<Table columns={columns} dataSource={data} loading={loading} />
		</div>
	);
};

export { RepoManage };
