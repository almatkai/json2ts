import { defineConfig } from 'vitest/config'
export default defineConfig({test:{include:['src/renderer/src/raycast-api/form-runtime.test.ts','contract.test.tsx'],environment:'node'},esbuild:{jsx:'automatic'}})
