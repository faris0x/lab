export interface MathFieldHandle {
  insert(latex: string): void;
  backspace(): void;
  clear(): void;
  focus(): void;
  getValue(): string;
}

let active: MathFieldHandle | null = null;

export function activateField(handle: MathFieldHandle): void {
  active = handle;
}

export function deactivateField(handle: MathFieldHandle): void {
  if (active === handle) active = null;
}

export function hasActiveField(): boolean {
  return active !== null;
}

export function insertActive(latex: string): void {
  active?.insert(latex);
}

export function backspaceActive(): void {
  active?.backspace();
}

export function clearActive(): void {
  active?.clear();
}

export function focusActive(): void {
  active?.focus();
}
