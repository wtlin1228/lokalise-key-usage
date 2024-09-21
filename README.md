# Lokalise Key Usage

This tool can find all usages of lokalise keys. The output of this tool will be the input of dependency tracker.

⚠️ This implementation only works for the following syntax, you could write your own to support your API or translation mark.

## Cases

1. for `const LABELS = translate(OBJ)`, should trace the usage of `LABELS`, like:

   - `LABELS.a`
   - `LABELS.a.b`
   - `LABELS[key]`
   - `LABELS.a[key]`

2. for `const topLevelSymbol = translate(<String Literal>)`, just bind the `<String Literal>` it into its top level scopped symbol.

3. for `const topLevelSymbol = translate(<String Literal>, { /* ... */ })`, just bind the `<String Literal>` it into its top level scopped symbol.

4. for `const topLevelSymbol = translate(<Expression>)`, if the `<Expression>` can be evaluated at build time and is a string, bind it into its top level scopped symbol.

   evaluate: https://swc.rs/docs/configuration/minification#swcminifycode-options:~:text=evaluate%2C%20defaults%20to%20true.

   Babel has something like "try to evaluate", let's find out how to do the same in SWC.

   Yes, it works.

   input:

   ```js
   {
     let a = 1;

     if (a === 1) {
       console.log(a);
     }

     const LOKALISE_KEY = "lokalise.key.cat";
     const bar = translate(LOKALISE_KEY);

     const LABEL_KEYS = {
       a: {
         b: {
           c: "lokalise.key.cat",
         },
       },
     };
     const boo = translate(LABEL_KEYS.a.b.c);
   }
   ```

   output:

   ```js
   {
     if (true) console.log(1);
     const bar = translate("lokalise.key.cat");
     const boo = translate("lokalise.key.cat");
   }
   ```

## Code Exmaples

### Simple

```jsx
const LABELS = translate({
  bird: "lokalise.key.bird",
  cat: "lokalise.key.cat",
  dog: "lokalise.key.dog",
});

const Foo = () => {
  return <div>{LABELS.bird}</div>;
};
```

- "lokalise.key.bird" -> `<Foo>`

### Conditional

```jsx
const LABELS = translate({
  bird: "lokalise.key.bird",
  cat: "lokalise.key.cat",
  dog: "lokalise.key.dog",
});

const Foo = (cond: boolean) => {
  return <div>{cond ? LABELS.bird : LABELS.cat}</div>;
};
```

- "lokalise.key.bird" -> `<Foo>`
- "lokalise.key.cat" -> `<Foo>`

### Computed Key

```tsx
const LABELS = translate({
  a: {
    b: {
      c: {
        cat: "lokalise.key.cat",
        dog: "lokalise.key.dog",
        bird: "lokalise.key.bird",
      },
    },
  },
});

const Foo = (type: "cat" | "bird") => {
  return <div>{LABELS.a.b.c[type]}</div>;
};
```

- "lokalise.key.bird" -> `<Foo>`
- "lokalise.key.cat" -> `<Foo>`
- "lokalise.key.dog" -> `<Foo>`

### Pass Object, Not Value

```jsx
const LABELS = translate({
  a: {
    b: {
      bird: "lokalise.key.bird",
    },
    c: {
      cat: "lokalise.key.cat",
    },
  },
  dog: "lokalise.key.dog",
});

const Foo = () => {
  return <Bar labels={LABELS.a} />;
};
```

- "lokalise.key.bird" -> `<Foo>`
- "lokalise.key.cat" -> `<Foo>`

### <Trans>

```jsx
const LABEL_KEYS = {
  cat: "lokalise.key.cat",
};

const Foo = () => <Trans i18nKey={LABEL_KEYS.cat} />;

const StyledTrans = styled(Trans)``;
const Bar = () => <StyledTrans i18nKey={LABEL_KEYS.cat} />;
```

- "lokalise.key.cat" -> `<Foo>`
- "lokalise.key.cat" -> `<Bar>`

### <TransBlock>

```jsx
const LABEL_KEYS = {
  cat: "lokalise.key.cat",
};

const Foo = () => <TransBlock i18nKey={LABEL_KEYS.cat} />;

const StyledTransBlock = styled(Trans)``;
const Bar = () => <StyledTransBlock i18nKey={LABEL_KEYS.cat} />;
```

- "lokalise.key.cat" -> `<Foo>`
- "lokalise.key.cat" -> `<Bar>`

### Testing

Should be ignored.

### Import

```js
export const LABELS = translate({
  bird: "lokalise.key.bird",
  cat: "lokalise.key.cat",
  dog: "lokalise.key.dog",
});
```

```jsx
import { LABELS } from "./labels";

const Foo = () => <div>{LABELS.bird}</div>;
```

- "lokalise.key.bird" -> `<Foo>`

### Directly

```js
const MY_LABEL = translate("lokalise.key.bird");

const Foo = () => <div>{translate("lokalise.key.bird")}</div>;
```

"lokalise.key.bird" -> `MY_LABEL`
"lokalise.key.bird" -> `<Foo>`

### CSS

```jsx
const box = styled.div`
  transform: translate(0, 50%);
`;
```
