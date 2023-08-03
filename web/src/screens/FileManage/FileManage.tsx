import { Input, Tree } from "antd";
import { post } from "../../useQuery";
import { useEffect, useState } from "react";
import { Row } from "../../layouts";

async function postLs(path: string) {
	// rome-ignore lint/suspicious/noExplicitAny: <explanation>
	const res: any[] = await post("/api/fs/ls", {
		json: {
			path,
		},
	});
	// rome-ignore lint/suspicious/noExplicitAny: <explanation>
	return res.map((it: any) => ({
		title: it.name,
		key: `${path}/${it.name}`,
		isLeaf: !it.is_dir,
	}));
}

interface DataNode {
	title: string;
	key: string;
	isLeaf?: boolean;
	children?: DataNode[];
}

const updateTreeData = (
	list: DataNode[],
	key: React.Key,
	children: DataNode[],
): DataNode[] =>
	list.map((node) => {
		if (node.key === key) {
			return {
				...node,
				children,
			};
		}
		if (node.children) {
			return {
				...node,
				children: updateTreeData(node.children, key, children),
			};
		}
		return node;
	});

export function FileManage() {
	const [treeData, setTreeData] = useState<DataNode[]>([]);

	useEffect(() => {
		(async () => {
			const child = await postLs("local");
			setTreeData(child);
		})();
	}, []);

	return (
		<Row
			style={{
				flex: 1,
				padding: 8,
			}}
		>
			<Tree
				treeData={treeData}
				loadData={async ({ key, children }) => {
					if (children) {
						return;
					}
					const child = await postLs(key);
					setTreeData((origin) => updateTreeData(origin, key, child));
				}}
				style={{
					height: "100%",
					borderWidth: 1,
					borderColor: "black",
					borderRadius: 5,
					borderStyle: "solid",
				}}
			/>
			<Input.TextArea rows={20} style={{ marginLeft: 8 }} />
		</Row>
	);
}
