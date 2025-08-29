import type { Song } from "@lib/models";
import type {
	SongWorkerRequest,
	SongWorkerResponse,
} from "@lib/worker-messages";

function sendMessage(message: SongWorkerResponse) {
	postMessage(message);
}

let songs: Array<Song> = [];

// @biome-ignore
onmessage = (event: MessageEvent<SongWorkerRequest>) => {
	const { type } = event.data;

	switch (type) {
		case "initialize":
			songs = event.data.payload;
			sendMessage({
				type,
				payload: songs,
			});
			break;
		case "groupArtists":
			sendMessage({
				type,
				payload: new Map(
					songs
						.map((song) => song.artist)
						.map((artist) => artist || "Unknown")
						.map((artist) => [
							artist as string,
							songs.filter((song) => {
								if (artist === "Unknown") {
									return song.artist === undefined;
								}

								return song.artist === artist;
							}),
						]),
				),
			});
			break;
		case "groupAlbums":
			sendMessage({
				type,
				payload: new Map(
					songs
						.map((song) => song.album)
						.map((album) => album || "Unknown")
						.map((album) => [
							album as string,
							songs.filter((song) => {
								if (album === "Unknown") {
									return song.album === undefined;
								}
								return song.album === album;
							}),
						]),
				),
			});
			break;
	}
};
