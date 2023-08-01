import { useEffect, useState } from "react";

export function useQuery(input: RequestInfo | URL, init?: RequestInit) {
	const [state, setState] = useState<{
		// rome-ignore lint/suspicious/noExplicitAny: <explanation>
		data?: any;
		// rome-ignore lint/suspicious/noExplicitAny: <explanation>
		error?: any;
	}>({});
	const [loading, setLoading] = useState(true);

	useEffect(() => {
		(async () => {
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
		})();
	}, []);

	return { ...state, loading };
}
