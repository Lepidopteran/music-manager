export function formatBytes(bytes: number | bigint, decimals = 2) {
	if (typeof bytes !== 'number' && typeof bytes !== 'bigint') return "0 Bytes";

	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes = [
		"Bytes",
		"KiB",
		"MiB",
		"GiB",
		"TiB",
		"PiB",
		"EiB",
		"ZiB",
		"YiB",
	];

	const i = Math.floor(Number(bytes) === 0 ? 0 : Math.log(Number(bytes)) / Math.log(k));
	const formattedBytes = (Number(bytes) / k ** i).toFixed(dm);

	return `${Number.parseFloat(formattedBytes)} ${sizes[i]}`;
}

