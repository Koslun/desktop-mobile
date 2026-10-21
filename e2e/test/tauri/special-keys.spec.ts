import process from 'node:process';
import { browser, expect } from '@wdio/globals';
import { Key } from 'webdriverio';
import '@wdio/native-types';

/**
 * WebDriver special keys (U+E000–U+E05D) through the embedded provider's own key mapping (#735).
 * The other providers hand keys to their upstream drivers, so this suite runs on embedded only.
 */

const driverProvider = process.env.DRIVER_PROVIDER as 'official' | 'crabnebula' | 'embedded' | 'external' | undefined;

const CONTROL_RIGHT = '';

type KeyEvent = {
  key: string;
  code: string;
  keyCode: number;
  shiftKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  metaKey: boolean;
};

const noModifiers = { shiftKey: false, ctrlKey: false, altKey: false, metaKey: false };

async function keyTargetEvents() {
  return (await browser.execute('return window.__keyTargetEvents || []')) as unknown as KeyEvent[];
}

async function editorText(selector = '#keys-editor') {
  const text = (await browser.execute(`return document.querySelector("${selector}").textContent`)) as string;
  return text.replace(/ /g, ' ');
}

describe('special keys (embedded)', () => {
  before(function () {
    if (driverProvider !== 'embedded') {
      this.skip();
    }
  });

  beforeEach(async () => {
    await browser.execute(
      'window.__keyTargetEvents = [];' +
        'document.querySelector("#keys-input").value = "";' +
        'document.querySelector("#keys-editor").textContent = "";' +
        'document.querySelector("#keys-blocking-input").value = "";' +
        'document.querySelector("#keys-blocking-editor").textContent = "";',
    );
  });

  it('should deliver PageDown, PageUp, Home and End to the focused element', async () => {
    await browser.$('#key-target').click();
    await browser.execute('window.__keyTargetEvents = [];');

    for (const key of [Key.PageDown, Key.PageUp, Key.Home, Key.End]) {
      await browser.keys(key);
    }

    expect(await keyTargetEvents()).toEqual([
      { key: 'PageDown', code: 'PageDown', keyCode: 34, ...noModifiers },
      { key: 'PageUp', code: 'PageUp', keyCode: 33, ...noModifiers },
      { key: 'Home', code: 'Home', keyCode: 36, ...noModifiers },
      { key: 'End', code: 'End', keyCode: 35, ...noModifiers },
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

  it('should not type the null key', async () => {
    const input = browser.$('#keys-input');
    await input.addValue(`a${Key.NULL}b`);
    expect(await input.getValue()).toBe('ab');
    await input.addValue(Key.NULL);
    expect(await input.getValue()).toBe('ab');
  });

  it('should replace a selection made with Control+A in an input', async () => {
    const input = browser.$('#keys-input');
    await input.setValue('old');
    await input.addValue(`${Key.Control}a${Key.NULL}new`);
    expect(await input.getValue()).toBe('new');
  });

  it('should replace a selection made with Control+A in a contenteditable', async () => {
    await browser.execute('document.querySelector("#keys-editor").textContent = "old";');
    await browser.$('#keys-editor').addValue(`${Key.Control}a${Key.NULL}new`);
    expect(await editorText()).toBe('new');
  });

  it('should carry held modifiers on special keys', async () => {
    await browser.$('#key-target').addValue(`${Key.Shift}${Key.Enter}`);
    expect((await keyTargetEvents()).find((event) => event.key === 'Enter')).toMatchObject({ shiftKey: true });

    await browser.execute('window.__keyTargetEvents = [];');
    await browser.keys([Key.Shift, Key.PageDown]);
    expect((await keyTargetEvents()).find((event) => event.key === 'PageDown')).toMatchObject({ shiftKey: true });
  });

  it('should continue after existing contenteditable content', async () => {
    await browser.execute('document.querySelector("#keys-editor").textContent = "old";');
    await browser.$('#keys-editor').addValue(`new${Key.PageDown}`);
    expect(await editorText()).toBe('oldnew');
  });

  it('should type printable special keys into a contenteditable', async () => {
    await browser.$('#keys-editor').addValue(`a${Key.Space}b`);
    expect(await editorText()).toBe('a b');
  });

  it('should not type or delete in an input when the keydown is canceled', async () => {
    await browser.execute('document.querySelector("#keys-blocking-input").value = "ab";');
    const input = browser.$('#keys-blocking-input');
    await input.addValue(`c${Key.Space}`);
    expect(await input.getValue()).toBe('abc');
    await input.addValue(Key.Backspace);
    expect(await input.getValue()).toBe('abc');
    await browser.keys('x');
    await browser.keys('y');
    expect(await input.getValue()).toBe('abcy');
  });

  it('should not type in a contenteditable when the keydown is canceled', async () => {
    await browser.execute('document.querySelector("#keys-blocking-editor").textContent = "ab";');
    await browser.$('#keys-blocking-editor').addValue(`c${Key.Space}`);
    expect(await editorText('#keys-blocking-editor')).toBe('abc');
    await browser.keys('x');
    await browser.keys('y');
    expect(await editorText('#keys-blocking-editor')).toBe('abcy');
  });

  it('should keep Control held while the other hand still holds it', async () => {
    const input = browser.$('#keys-input');
    await input.setValue('old');
    await input.click();
    await browser
      .action('key')
      .down(Key.Control)
      .down(CONTROL_RIGHT)
      .up(CONTROL_RIGHT)
      .down('a')
      .up('a')
      .up(Key.Control)
      .perform();

    const selection = await browser.execute(
      'var el = document.querySelector("#keys-input"); return [el.value, el.selectionStart, el.selectionEnd];',
    );
    expect(selection).toEqual(['old', 0, 3]);
  });
});
