import { StandardSchemaV1Issue } from "@tanstack/react-form";
import { AlertCircle } from "lucide-react";
import { useMemo } from "react";

interface ErrorDisplayProps {
  errors?:
    | string
    | StandardSchemaV1Issue
    | Array<string | StandardSchemaV1Issue | undefined>;
}

export function FormFieldErrors({ errors }: ErrorDisplayProps) {
  const errorList = useMemo(
    () =>
      (Array.isArray(errors) ? errors : [errors]).filter(Boolean) as Array<
        string | StandardSchemaV1Issue
      >,
    [errors]
  );

  if (!errorList.length) {
    return null;
  }

  return (
    <ul className="mt-1 space-y-1 text-sm text-destructive">
      {errorList.map((error, index) => {
        if (typeof error === "string") {
          return (
            <li key={index} className="flex items-center space-x-2">
              <AlertCircle
                size={16}
                className="flex-shrink-0 text-destructive"
              />
              <span>{error}</span>
            </li>
          );
        }
        return (
          <li key={index} className="flex flex-col space-y-0.5">
            <div className="flex items-center space-x-2">
              <AlertCircle
                size={16}
                className="flex-shrink-0 text-destructive"
              />
              <span>{error.message}</span>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
