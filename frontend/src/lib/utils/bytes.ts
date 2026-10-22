export interface FormatBytesOptions {
	decimals?: number;
	fullName?: boolean;
}

interface ByteFormat {
	short: string;
	full: string;
}

export function formatBytes(bytes: number | bigint, options: FormatBytesOptions = {}) {
	const { decimals = 2, fullName = false } = options;
	if (typeof bytes !== "number" && typeof bytes !== "bigint") return fullName ? "0 Bytes" : "0 B";

	const k = 1024;
	const dm = decimals < 0 ? 0 : decimals;
	const sizes: ByteFormat[] = [
		{ short: "B", full: "Bytes" },
		{ short: "KiB", full: "Kilobytes" },
		{ short: "MiB", full: "Megabytes" },
		{ short: "GiB", full: "Gigabytes" },
		{ short: "TiB", full: "Terabytes" },
		{ short: "PiB", full: "Petabytes" },
		{ short: "EiB", full: "Exabytes" },
		{ short: "ZiB", full: "Zettabytes" },
		{ short: "YiB", full: "Yottabytes" },
	];

	const i = Math.floor(Number(bytes) === 0 ? 0 : Math.log(Number(bytes)) / Math.log(k));
	const formattedBytes = (Number(bytes) / k ** i).toFixed(dm);

	return `${Number.parseFloat(formattedBytes)} ${fullName ? sizes[i].full : sizes[i].short}`;
}
