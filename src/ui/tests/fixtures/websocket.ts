import { APIRequestContext, request } from '@playwright/test';

export async function publishToWebsocket(channel: string, data: any) {
  const context = await request.newContext({
    ignoreHTTPSErrors: true,
  });
  const response = await context.post('https://localhost:8000/api/internal/publish', {
    headers: {
      'Content-Type': 'application/json',
    },
    data: {
      channel,
      data,
    },
  });

  if (!response.ok()) {
    console.error(`Failed to publish to WebSocket: ${response.status()} ${response.statusText()}`);
    console.error(await response.text());
    throw new Error('WebSocket publish failed');
  }
}
