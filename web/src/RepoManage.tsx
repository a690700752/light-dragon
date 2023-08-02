import { Button, Input, Modal, Space, Table, notification } from "antd";
import { ColumnsType } from "antd/es/table";
import React from "react";
import { Column, Row } from "./layouts";
import { post, useQuery } from "./useQuery";

function useAddModal(onAdd: () => void) {
	const [isAddModalVisible, setIsAddModalVisible] = React.useState(false);

	const show = () => setIsAddModalVisible(true);
	const dismiss = () => setIsAddModalVisible(false);

	const inputsRef = React.useRef<{
		repo?: string;
		branch: string;
		schedule: string;
		whitelist: string;
	}>({
		repo: "https://github.com/a690700752/jdpro",
		branch: "master",
		schedule: "5 10 * * *",
		whitelist: ".*\\.ts$",
	});

	const renderModal = () => (
		<Modal
			title="Add Repo"
			open={isAddModalVisible}
			onOk={async () => {
				try {
					await post("/api/repo/add", {
						json: inputsRef.current,
					});

					dismiss();
					onAdd();
					// rome-ignore lint/suspicious/noExplicitAny: <explanation>
				} catch (e: any) {
					console.log("e", e);
					notification.error({
						message: "Error",
						description: `Failed to add repo ${e}`,
					});
				}
			}}
			onCancel={dismiss}
		>
			<Space direction="vertical" style={{ width: "100%" }}>
				<span>
					<span style={{ color: "red" }}>* </span>Repo ( url for git repo or
					name for local repo)
				</span>
				<Input
					onChange={(e) => {
						inputsRef.current.repo = e.target.value;
					}}
				/>
				<span>Branch</span>
				<Input
					defaultValue={inputsRef.current.branch}
					onChange={(e) => {
						inputsRef.current.branch = e.target.value;
					}}
				/>
				<span>schedule</span>
				<Input
					defaultValue={inputsRef.current.schedule}
					onChange={(e) => {
						inputsRef.current.schedule = e.target.value;
					}}
				/>
				<span>whitelist</span>
				<Input
					defaultValue={inputsRef.current.whitelist}
					onChange={(e) => {
						inputsRef.current.whitelist = e.target.value;
					}}
				/>
			</Space>
		</Modal>
	);

	return {
		renderModal,
		show,
		dismiss,
	};
}

function getColumns(refresh: () => void) {
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
			render: (_, record, index) => (
				<Space size="middle">
					<Button
						onClick={async () => {
							try {
								await post("/api/repo/rm", {
									json: {
										index,
									},
								});
								refresh();
								notification.success({
									message: "Success",
									description: "delete success",
								});
								// rome-ignore lint/suspicious/noExplicitAny: <explanation>
							} catch (e: any) {
								notification.error({
									message: "Error",
									description: `Failed to delete repo ${e}`,
								});
							}
						}}
					>
						Delete
					</Button>
				</Space>
			),
		},
	];

	return columns;
}

const RepoManage: React.FC = () => {
	const { data, loading, refresh } = useQuery("/api/repo/list", {
		method: "POST",
	});

	const { show, renderModal } = useAddModal(() => {
		refresh();
		notification.success({
			message: "Success",
			description: "add success",
		});
	});

	return (
		<>
			<Column>
				<Row>
					<Button
						onClick={() => {
							show();
						}}
					>
						Add
					</Button>
				</Row>
				<Table
					style={{ marginTop: 8 }}
					columns={getColumns(refresh)}
					dataSource={data}
					loading={loading}
				/>
			</Column>
			{renderModal()}
		</>
	);
};

export { RepoManage };
