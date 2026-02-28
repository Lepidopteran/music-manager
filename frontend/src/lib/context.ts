import type { GroupedSongs } from "@lib/state/group";
import type { GroupKey } from "@lib/workers";
import { createContext } from "svelte";

export interface SongGroups extends Partial<Record<GroupKey, GroupedSongs>> {
	tracked: GroupKey[];
	inProgress: GroupKey[];
}

export const [songGroups, setSongGroups] = createContext<SongGroups>();
