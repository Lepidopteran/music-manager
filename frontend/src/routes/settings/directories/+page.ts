import { getDirectories } from "@api/directory";
import type { PageData } from "../../$types";

export const load: PageData = async () => {
	return { directories: await getDirectories() };
};
