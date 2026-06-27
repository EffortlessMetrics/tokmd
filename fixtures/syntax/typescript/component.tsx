import React from "react";

export type NativeButtonProps = {
  label: string;
  onClick?: () => void;
};

export function NativeButton(props: NativeButtonProps) {
  const handle = props.onClick as () => void;
  return <button onClick={() => handle?.()}>{props.label}</button>;
}

export default function App() {
  return <NativeButton label="Run" />;
}
