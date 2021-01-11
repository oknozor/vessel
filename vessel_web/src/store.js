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

    eventSource.onmessage = e => {
        const eventPayload = e.data
            .replaceAll("\\\"", "\"")
            .replaceAll("\\n", "");

        // DIRTY HANDLING OF EVENT DATA
        let title;

        // DIRTY END

        const logElement = {
            title: "test",
            body: eventPayload,
            date: moment().format('LL'),
            hour: moment().format('LT')
        };

        update(messages => messages.concat(logElement));
    };

    return {
        subscribe,
        reset: () => set([]),
        close: eventSource.close,
    };
};