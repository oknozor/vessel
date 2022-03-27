import { writable } from 'svelte/store';

const eventSource = new EventSource(
    `http://127.0.0.1:3031/events`,
);

eventSource.onerror = e => {
    console.log("Event source is broken", e);
}

export const createSearchStore = () => {
    const {subscribe, set, update} = writable([])

    const handler = e => {
        update(messages => messages.concat(JSON.parse(e.data)));
    };

    const reset = () => set([]);

    const close = () => eventSource.removeEventListener("search_reply", handler)

    eventSource.addEventListener("search_reply", handler);

    return {
        subscribe,
        reset,
        close,
    }
};

export const createDownloadStore = () => {
    const {subscribe, set, update} = writable({})

    const handler = e => {
        update(_ => JSON.parse(e.data));
    };

    const reset = () => set({});

    const close = () => eventSource.removeEventListener("download_started", handler)

    eventSource.addEventListener("download_started", handler);

    return {
        subscribe,
        reset,
        close,
    }
}

export const createRoomListStore = () => {
    const {subscribe, set, update} = writable({})

    const handler = e => {
        update(_ => JSON.parse(e.data));
    };

    const reset = () => set({
        rooms: [],
        owned_private_rooms: [],
        private_rooms: [],
        operated_private_rooms: [],
    });

    const close = () => eventSource.removeEventListener("room_lists", handler)

    eventSource.addEventListener("room_lists", handler);

    return {
        subscribe,
        reset,
        close,
    }
};

export const createDownloadProgressStore = () => {
    const {subscribe, set, update} = writable({})

    const handler = e => {
        update(_ => JSON.parse(e.data));
    };

    const reset = () => set({});

    const close = () => eventSource.removeEventListener("download_progress", handler)

    eventSource.addEventListener("download_progress", handler);

    return {
        subscribe,
        reset,
        close,
    }
};