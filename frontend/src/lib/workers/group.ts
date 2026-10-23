import type { DatabaseSong } from "#bindings/DatabaseSong";
import { sendMessage } from "./utils";

type GroupKey = keyof DatabaseSong;
type GroupedSongs = Record<string, Array<DatabaseSong>>;

function groupSongs(groupKey: GroupKey, songs: Array<DatabaseSong>): GroupedSongs {
	return Object.fromEntries(
		songs.reduce((map, song) => {
			const key = (song[groupKey]) as string;
			const group = map.get(key);

			if (!key) {
				return map;
			}

			if (group) {
				group.push(song);
			} else {
				map.set(key, [song]);
			}

			return map;
		}, new Map<string, Array<DatabaseSong>>()).entries(),
	);
}

onmessage = (event: MessageEvent<GroupWorkerRequest>) => {
	const { key, songs } = event.data;
	sendMessage({
		key,
		grouped: groupSongs(key, songs),
	});
};

export type GroupWorkerRequest = {
	key: GroupKey;
	songs: Array<DatabaseSong>;
};

export type GroupWorkerResponse = {
	key: GroupKey;
	grouped: GroupedSongs;
};
