const LABELS = translate({
  bird: "i18n.bird",
  cat: "i18n.cat",
  dog: "i18n.dog",
});

function A() {
  return (
    <div>
      <p>{LABELS.bird}</p>
      <p>{LABELS.cat}</p>
    </div>
  );
}

function B() {
  return (
    <div>
      <p>{LABELS.bird}</p>
      <p>{LABELS.cat}</p>
    </div>
  );
}

function C() {
  return (
    <div>
      <p>{LABELS.cat}</p>
    </div>
  );
}
