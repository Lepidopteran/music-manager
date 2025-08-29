function compare<T>(
	x: T,
	y: T,
	leftChain: unknown[] = [],
	rightChain: unknown[] = [],
	depth: number = Infinity,
): boolean {
	if (depth < 0) return true;

	if (x === y) return true;
	if (typeof x === "number" && typeof y === "number" && isNaN(x) && isNaN(y)) {
		return true;
	}

	if (
		(typeof x === "function" && typeof y === "function") ||
		(x instanceof Date && y instanceof Date) ||
		(x instanceof RegExp && y instanceof RegExp) ||
		(x instanceof String && y instanceof String) ||
		(x instanceof Number && y instanceof Number)
	) {
		return x.toString() === y.toString();
	}

	if (!(x instanceof Object && y instanceof Object)) return false;
	if (x.isPrototypeOf?.(y) || y.isPrototypeOf?.(x)) return false;

	if (
		x.constructor !== y.constructor ||
		(x as any).prototype !== (y as any).prototype
	) {
		return false;
	}

	if (leftChain.includes(x) || rightChain.includes(y)) return false;

	const keys = Array.from(
		new Set([...Object.keys(x as object), ...Object.keys(y as object)]),
	);

	for (const key of keys) {
		const valX = (x as any)[key];
		const valY = (y as any)[key];

		if (typeof valX !== typeof valY) return false;

		if ((typeof valX === "object" || typeof valX === "function") && depth > 0) {
			leftChain.push(valX);
			rightChain.push(valY);
			if (!compare(valX, valY, leftChain, rightChain, depth - 1)) return false;
			leftChain.pop();
			rightChain.pop();
		} else if (valX !== valY) {
			return false;
		}
	}

	return true;
}

export function deepCompare<T>(
	a: T,
	b: T,
	maxDepth: number = Infinity,
): boolean {
	return compare(a, b, [], [], maxDepth);
}
