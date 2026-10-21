import process from 'node:process';
import { browser, expect } from '@wdio/globals';
import { Key } from 'webdriverio';
import '@wdio/native-types';

/**
 * WebDriver special keys (U+E000–U+E05D) through the embedded provider's own key mapping (#735).
 * The other providers hand keys to their upstream drivers, so this suite runs on embedded only.
 */

const driverProvider = process.env.DRIVER_PROVIDER as 'official' | 'crabnebula' | 'embedded' | 'external' | undefined;

type KeyEvent = {
  key: string;
  code: string;
  keyCode: number;
};

describe('special keys (embedded)', () => {
  before(function () {
    if (driverProvider !== 'embedded') {
      this.skip();
    }
  });

  beforeEach(async () => {
    await browser.execute(
      'window.__keyTargetEvents = []; var el = document.querySelector("#keys-input"); if (el) { el.value = ""; }',
    );
  });

  it('should deliver PageDown, PageUp, Home and End to the focused element', async () => {
    await browser.$('#key-target').click();
    await browser.execute('window.__keyTargetEvents = [];');

    for (const key of [Key.PageDown, Key.PageUp, Key.Home, Key.End]) {
      await browser.keys(key);
    }

    const events = (await browser.execute('return window.__keyTargetEvents || []')) as unknown as KeyEvent[];
    expect(events).toEqual([
      { key: 'PageDown', code: 'PageDown', keyCode: 34 },
      { key: 'PageUp', code: 'PageUp', keyCode: 33 },
      { key: 'Home', code: 'Home', keyCode: 36 },
      { key: 'End', code: 'End', keyCode: 35 },
    ]);
  });

  it('should press special keys inside addValue instead of typing them', async () => {
    const input = browser.$('#keys-input');
    await input.addValue(`abc${Key.Backspace}d${Key.PageDown}`);
    expect(await input.getValue()).toBe('abd');
  });

  it('should type the digit for a numpad key inside addValue', async () => {
    const input = browser.$('#keys-input');
    await input.addValue(`1${Key.Numpad2}`);
    expect(await input.getValue()).toBe('12');
  });
});
