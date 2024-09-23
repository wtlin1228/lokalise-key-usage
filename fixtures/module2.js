const LABELS = translate({
  bird: "i18n.bird",
  cat: "i18n.cat",
  dog: "i18n.dog",
});

function Foo() {
  return (
    <div>
      <p>{LABELS.bird}</p>
    </div>
  );
}

function Bar() {
  return (
    <div>
      <p>{LABELS.bird}</p>
    </div>
  );
}
