# Lokalise Key Usage

This tool can find all usages of lokalise keys. The output of this tool will be the input of dependency tracker.

## Cases

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

- "lokalise.key.bird" -> <Foo>

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

- "lokalise.key.bird" -> <Foo>
- "lokalise.key.cat" -> <Foo>

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

- "lokalise.key.bird" -> <Foo>
- "lokalise.key.cat" -> <Foo>
- "lokalise.key.dog" -> <Foo>

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

- "lokalise.key.bird" -> <Foo>
- "lokalise.key.cat" -> <Foo>

### <Trans>

```jsx
const LABEL_KEYS = {
  cat: "lokalise.key.cat",
};

const Foo = () => <Trans i18nKey={LABEL_KEYS.cat} />;
```

- "lokalise.key.cat" -> <Foo>

### <TransBlock>

```jsx
const LABEL_KEYS = {
  cat: "lokalise.key.cat",
};

const Foo = () => <TransBlock i18nKey={LABEL_KEYS.cat} />;
```

- "lokalise.key.cat" -> <Foo>

### Testing

Should be ignored.
