import { writable } from 'svelte/store';
import moment from "moment/moment";

export const createChannelStore = () => {
    const { subscribe, set, update } = writable([]);

    const eventSource = new EventSource(
        `http://127.0.0.1:3031/events`,
    );

    eventSource.onerror = e => {
        console.log("Event source is broken", e);
    }

    eventSource.addEventListener("search_reply", e => {
        console.log(e.type)
        const eventPayload = e.data
            .replaceAll("\\\"", "\"")
            .replaceAll("\\n", "");
        let json = JSON.parse(e.data);

        console.log(json)

        update(messages => messages.concat(json));
    });

    return {
        subscribe,
        reset: () => set([]),
        close: eventSource.close,
    };
};