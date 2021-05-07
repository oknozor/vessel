import { writable } from 'svelte/store';

export const searchSub = writable([]);
export const dlStartedSub = writable([]);
export const dlProgressSub = writable(new Map);

const eventSource = new EventSource(
    `http://127.0.0.1:3031/events`,
);

eventSource.onerror = e => {
    console.log("Event source is broken", e);
}

export const createSearchStore = () => {
    eventSource.addEventListener("search_reply", e => {
        let json = JSON.parse(e.data);
        searchSub.update(messages => messages.concat(json));
    });

    return searchSub
};

export const createDownloadStore = () => {
    eventSource.addEventListener("download_started", e => {
        let json = JSON.parse(e.data);
        dlStartedSub.update(messages => messages.concat(json));
    });

    return dlStartedSub
};

export const createDownloadProgressStore = () => {
    eventSource.addEventListener("download_progress", e => {
        let json = JSON.parse(e.data);
        dlProgressSub.update(messages => messages.set(json.ticket, json.percent));
    });

    return dlProgressSub
};