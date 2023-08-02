import { useEffect, useState } from "react";

export function usePost(
	input: RequestInfo | URL,
	init?: RequestInit & {
		json?: object;
		// rome-ignore lint/suspicious/noExplicitAny: <explanation>
		mapData?: (data: any) => any;
	},
) {
	const { mapData = (i: unknown) => i, ...restInit } = init || {};
	const [state, setState] = useState<{
		// rome-ignore lint/suspicious/noExplicitAny: <explanation>
		data?: any;
		// rome-ignore lint/suspicious/noExplicitAny: <explanation>
		error?: any;
	}>({});
	const [loading, setLoading] = useState(true);

	const refresh = async () => {
		setLoading(true);
		try {
			const data = await post(input, restInit);
			setState({
				data: mapData(data),
			});
		} catch (e) {
			setState({
				error: e,
			});
		}
		setLoading(false);
	};

	useEffect(() => {
		refresh();
	}, []);

	return { ...state, loading, refresh };
}

export async function post(
	input: RequestInfo | URL,
	init?: RequestInit & {
		json?: object;
	},
) {
	const json = init?.json;
	const res = await fetch(input, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		...init,
		body: json ? JSON.stringify(json) : undefined,
	});
	const resJson = await res.json();
	console.log("resJson", resJson);
	if (resJson.code === 200) {
		return resJson.data;
	} else {
		throw new Error(JSON.stringify(resJson));
	}
}
