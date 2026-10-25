import type { AddressInfo } from 'node:net';
import { createServer } from 'node:net';
import { describe, expect, it } from 'vitest';
import { DevTool } from '../src/devTool.js';

async function closedPort(): Promise<number> {
  const server = createServer();
  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address() as AddressInfo;
  await new Promise<void>((resolve) => server.close(() => resolve()));
  return port;
}

describe('DevTool', () => {
  it('should name the refused address when the debugger port is closed', async () => {
    const port = await closedPort();
    const devTool = new DevTool({ host: 'localhost', port, timeout: 200 });

    await expect(devTool.list()).rejects.toMatchObject({
      message: expect.stringMatching(new RegExp(`^Request Error: .*ECONNREFUSED .*:${port}`)),
    });
  });
});
