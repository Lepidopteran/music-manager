import type { DatabaseSong } from "@lib/bindings/DatabaseSong";
import type { Song } from "@lib/models";
import { sendMessage } from "./utils";

type GroupKey = keyof DatabaseSong;
type GroupedSongs = Record<string, Array<Song>>;

function groupSongs(groupKey: GroupKey, songs: Array<Song>): GroupedSongs {
	return Object.fromEntries(
		songs.reduce((map, song) => {
			const key = (song[groupKey] ?? "Unknown") as string;
			const group = map.get(key);

			if (group) {
				group.push(song);
			} else {
				map.set(key, [song]);
			}

			return map;
		}, new Map<string, Array<Song>>()).entries(),
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
	songs: Array<Song>;
};

export type GroupWorkerResponse = {
	key: GroupKey;
	grouped: GroupedSongs;
};
