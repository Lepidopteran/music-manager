export const taskEventsSource = $state(new EventSource("/api/tasks/events"));
