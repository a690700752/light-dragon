import { useEffect, useState } from "react";

export function useQuery(input: RequestInfo | URL, init?: RequestInit) {
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
			const res = await fetch(input, init);
			setState({
				data: await res.json(),
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
		json: object;
	},
) {
	const json = init?.json;
	const res = await fetch(input, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		...init,
		body: JSON.stringify(json),
	});
	const resJson = await res.json();
	console.log("resJson", resJson);
	if (resJson.code === 200) {
		return resJson;
	} else {
		throw new Error(JSON.stringify(resJson));
	}
}
