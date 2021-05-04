import { writable } from 'svelte/store';

export const { subscribe, set, update } = writable([]);

export const createChannelStore = () => {
    const eventSource = new EventSource(
        `http://127.0.0.1:3031/events`,
    );

    eventSource.onerror = e => {
        console.log("Event source is broken", e);
    }

    eventSource.addEventListener("search_reply", e => {
        let json = JSON.parse(e.data);
        update(messages => messages.concat(json));
    });

    return {
        subscribe,
        reset: () => set([]),
        close: eventSource.close,
    };
};