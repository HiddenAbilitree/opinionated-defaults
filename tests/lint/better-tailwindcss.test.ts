import { expect, test } from 'bun:test';
import { ESLint } from 'eslint';
import { fileURLToPath } from 'node:url';

const configPath = fileURLToPath(
  new URL(`better-tailwindcss/eslint.config.ts`, import.meta.url),
);

const lintText = async (source: string) => {
  const eslint = new ESLint({ fix: true, overrideConfigFile: configPath });
  const results = await eslint.lintText(source, {
    filePath: fileURLToPath(
      new URL(`better-tailwindcss/test-file.tsx`, import.meta.url),
    ),
  });
  return results[0]?.output ?? source;
};

test(`better-tailwindcss: no recursive fix`, async () => {
  const source = `import Link from 'next/link';

export default function NotFound() {
  return (
    <section
      className='
      flex h-screen flex-col items-center justify-center gap-6
    '
    >
      <div className='flex flex-col items-center justify-center gap-1'>
        <h1 className='text-9xl'>404</h1>
        <p className='text-2xl'>Page not found</p>
      </div>
      <Link
        className='
          rounded-full border-5 border-tns-blue bg-tns-blue px-9 py-2
          text-tns-black
        '
        href='/'
      >
        Return to safety
      </Link>
    </section>
  );
}
`;

  const firstPass = await lintText(source);
  const secondPass = await lintText(firstPass);

  expect(secondPass).toBe(firstPass);
});
