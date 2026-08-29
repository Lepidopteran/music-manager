import { createContext } from "svelte";

export type StackItemProps = {
	index: number;
};

export const [stackItemProps, setStackItemProps] = createContext<StackItemProps>();
