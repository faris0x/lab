export type SolverModule = 'simplex' | 'linalg' | 'complexnum' | 'graphnet';

interface GeneratedModule {
  default: (moduleOrPath?: string | URL | Request) => Promise<unknown>;
  solve: (input: string) => string;
}

interface LoadedSolver {
  solve: (input: string) => string;
}

const generations = new Map<SolverModule, number>();
const loaded = new Map<SolverModule, Promise<LoadedSolver>>();

function baseUrl(): string {
  return import.meta.env.BASE_URL || '/';
}

function assetUrl(name: SolverModule, file: string, generation: number): string {
  const query = generation === 0 ? '' : `?v=${generation}`;
  return `${baseUrl()}wasm/${name}/${file}${query}`;
}

async function instantiate(name: SolverModule): Promise<LoadedSolver> {
  const generation = generations.get(name) ?? 0;
  const glueUrl = assetUrl(name, `${name}.js`, generation);
  const wasmUrl = assetUrl(name, `${name}_bg.wasm`, generation);
  const module = (await import(/* @vite-ignore */ glueUrl)) as GeneratedModule;
  await module.default(wasmUrl);
  return { solve: (input: string) => module.solve(input) };
}

export function loadSolver(name: SolverModule): Promise<LoadedSolver> {
  let promise = loaded.get(name);
  if (!promise) {
    promise = instantiate(name).catch((error) => {
      resetSolver(name);
      throw error;
    });
    loaded.set(name, promise);
  }
  return promise;
}

/** Warm a solver without blocking the first paint. */
export function preloadSolver(name: SolverModule): void {
  void loadSolver(name).catch(() => {});
}

/** Drop a cached instance so the next call loads a fresh module. */
export function resetSolver(name: SolverModule): void {
  loaded.delete(name);
  generations.set(name, (generations.get(name) ?? 0) + 1);
}

export async function solveJson<T>(
  name: SolverModule,
  input: unknown
): Promise<T> {
  const solver = await loadSolver(name);
  let json: string;
  try {
    json = solver.solve(JSON.stringify(input));
  } catch (error) {
    resetSolver(name);
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`The ${name} module failed: ${message}`);
  }
  try {
    return JSON.parse(json) as T;
  } catch {
    throw new Error(`The ${name} module returned an invalid response.`);
  }
}
