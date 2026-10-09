import { getDirectories } from "#api/directory";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
	const directories = await getDirectories();
	return { directories };
};
