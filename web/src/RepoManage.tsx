import { Button, Input, Modal, Space, Table, notification } from "antd";
import { ColumnsType } from "antd/es/table";
import React from "react";
import { Column, Row } from "./layouts";
import { post, usePost } from "./useQuery";

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
					defaultValue={inputsRef.current.repo}
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
				<span>update schedule</span>
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

function ListTasksModal(props: {
	name: string;
	onCancel: () => void;
}) {
	const { name, onCancel } = props;
	const { data, loading } = usePost("/api/repo/listTasks", {
		method: "POST",
		json: {
			name,
		},
		mapData: (d) => {
			// rome-ignore lint/suspicious/noExplicitAny: <explanation>
			d.forEach((element: any) => {
				element.name = element.args.Left.name;
			});
			return d;
		},
	});

	console.log("data", data);

	return (
		<Modal open={true} onCancel={onCancel} footer={null}>
			<Table
				dataSource={data}
				loading={loading}
				columns={[
					{
						title: "Name",
						dataIndex: "name",
						key: "name",
					},
					{
						title: "Run",
						dataIndex: "schedule",
						key: "schedule",
					},
				]}
			/>
		</Modal>
	);
}

const RepoManage: React.FC = () => {
	const { data, loading, refresh } = usePost("/api/repo/list", {
		method: "POST",
		mapData: (d) => {
			// rome-ignore lint/suspicious/noExplicitAny: <explanation>
			d.forEach((element: any) => {
				element.name = element.args.Left.name;
			});
			return d;
		},
	});

	const { show: showAdd, renderModal: renderAddModal } = useAddModal(() => {
		refresh();
		notification.success({
			message: "Success",
			description: "add success",
		});
	});

	const [taskListModal, setTaskListModal] = React.useState<{
		name: string;
		visible: boolean;
	}>({
		name: "",
		visible: false,
	});

	// rome-ignore lint/suspicious/noExplicitAny: <explanation>
	const columns: ColumnsType<any> = [
		{
			title: "Name",
			dataIndex: "name",
			key: "name",
		},
		{
			title: "Branch",
			dataIndex: ["args", "Left", "repo_args", "branch"],
			key: "branch",
		},
		{
			title: "Update",
			dataIndex: "schedule",
			key: "schedule",
		},
		{
			title: "Whitelist",
			dataIndex: ["args", "Left", "repo_args", "whitelist"],
			key: "whitelist",
		},
		{
			title: "Action",
			key: "action",
			render: (_, record, index) => (
				<Space size="middle">
					<Button
						onClick={() => {
							setTaskListModal({
								name: record.name,
								visible: true,
							});
						}}
					>
						Tasks
					</Button>
					<Button>Files</Button>
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

	return (
		<>
			<Column>
				<Row>
					<Button
						onClick={() => {
							showAdd();
						}}
					>
						Add
					</Button>
				</Row>
				<Table
					style={{ marginTop: 8 }}
					columns={columns}
					dataSource={data}
					loading={loading}
					rowKey="name"
				/>
			</Column>
			{renderAddModal()}
			{taskListModal.visible && (
				<ListTasksModal
					name={taskListModal.name}
					onCancel={() => {
						setTaskListModal({
							name: "",
							visible: false,
						});
					}}
				/>
			)}
		</>
	);
};

export { RepoManage };
