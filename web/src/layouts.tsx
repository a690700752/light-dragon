export function Row(props: React.HTMLAttributes<HTMLDivElement>) {
	return (
		<div
			{...props}
			style={{
				display: "flex",
				...props.style,
			}}
		/>
	);
}

export function Column(props: React.HTMLAttributes<HTMLDivElement>) {
	return (
		<div
			{...props}
			style={{
				display: "flex",
				flexDirection: "column",
				...props.style,
			}}
		/>
	);
}
